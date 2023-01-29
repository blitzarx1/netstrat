use std::collections::HashMap;

use egui::ScrollArea;
use tracing::error;

use crate::{
    netstrat::{Bus, Message, channels},
    widgets::{
        history::{History, Step},
        net_props::FrozenElements,
        AppWidget,
    },
};

use super::{
    messages::{MessageOperation, MessageOperationResult, OperationType},
    Controls,
};

const INFO_MSG_PLACEHOLDER: &str = "Press ▶ to start simulation";

pub struct SimulationProps {
    bus: Bus,
    step: Option<usize>,
    steps: HashMap<FrozenElements, String>,
    info_msg: String,
    history: Option<History>,
}

impl SimulationProps {
    pub fn new(bus: Bus) -> Self {
        Self {
            bus,
            info_msg: INFO_MSG_PLACEHOLDER.to_string(),
            history: Default::default(),
            step: Default::default(),
            steps: Default::default(),
        }
    }

    fn update(&mut self, controls: Controls) {
        self.handle_incoming_events();
        self.handle_controls(controls);
    }

    fn handle_controls(&mut self, controls: Controls) {
        let mut payload_operation = None;

        if controls.back_step_pressed {
            payload_operation = Some(OperationType::BackStep);
        }

        if controls.next_step_pressed {
            payload_operation = Some(OperationType::NextStep)
        }

        if controls.reset_pressed {
            payload_operation = Some(OperationType::Reset);
            self.info_msg = INFO_MSG_PLACEHOLDER.to_string();
        }

        if payload_operation.is_none() {
            return;
        }

        let msg = Message::new(
            serde_json::to_string(&MessageOperation::new(payload_operation.unwrap())).unwrap(),
        );

        if let Err(err) = self.bus.write(channels::SIMULATION_CHANNEL.to_string(), msg) {
            error!("failed to publish message: {err}");
        }
    }

    fn handle_incoming_events(&mut self) {
        if let Ok(msg) = self.bus.read(channels::SIMULATION_CHANNEL.to_string()) {
            let operation_result =
                serde_json::from_str::<MessageOperationResult>(&msg.payload()).unwrap();

            if operation_result.signal_holders().edges().is_empty()
                && operation_result.signal_holders().nodes().is_empty()
            {
                self.info_msg = INFO_MSG_PLACEHOLDER.to_string();
                self.step = Default::default();
                self.history = Default::default();
                self.steps = Default::default();
                return;
            }

            let step = match self.step {
                Some(step) => step + 1,
                None => 1,
            };

            self.step = Some(step);

            let msg = format!("Step: {step}");
            self.info_msg = msg.clone();

            // if self.history.is_none() {
            //     self.history = Some(History::new(msg.clone()));
            //     self.steps
            //         .insert(operation_result.signal_holders().clone(), msg);
            // } else {
            //     match self.steps.get(operation_result.signal_holders()) {
            //         Some(cycled_step_name) => {
            //             self.history
            //                 .as_mut()
            //                 .unwrap()
            //                 .cycle_and_set_step(cycled_step_name.clone());
            //         }
            //         None => {
            //             self.history
            //                 .as_mut()
            //                 .unwrap()
            //                 .add_and_set_current_step(step);
            //             self.steps
            //                 .insert(operation_result.signal_holders().clone(), msg);
            //         }
            //     }
            // }
        }
    }
}

impl AppWidget for SimulationProps {
    fn show(&mut self, ui: &mut egui::Ui) {
        let mut controls = Controls::default();

        ScrollArea::both()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    if ui.button("◀").clicked() {
                        controls.back_step_pressed = true
                    }
                    if ui.button("▶").clicked() {
                        controls.next_step_pressed = true
                    };
                    if ui.button("⟲").clicked() {
                        controls.reset_pressed = true
                    }
                });

                ui.separator();

                ui.label(self.info_msg.clone());

                self.update(controls);

                if self.history.is_some() {
                    ui.separator();
                    self.history.as_mut().unwrap().show(ui);
                }
            });
    }
}
