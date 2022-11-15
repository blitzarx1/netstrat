use tracing::error;

use crate::{
    netstrat::{Bus, Message},
    widgets::AppWidget,
};

use super::{
    messages::{MessageOperation, OperationType},
    Controls,
};

pub const SIMULATION_WIDGET_NAME: &str = "simulation";
const INFO_MSG_PLACEHOLDER: &str = "Press ▶ to start simulation";

pub struct SimulationProps {
    bus: Bus,
    step: Option<usize>,
    info_msg: String,
}

impl SimulationProps {
    pub fn new(bus: Bus) -> Self {
        Self {
            bus,
            info_msg: INFO_MSG_PLACEHOLDER.to_string(),
            step: Default::default(),
        }
    }

    fn update(&mut self, controls: Controls) {
        self.handle_incoming_events();
        self.handle_controls(controls);
    }

    fn handle_controls(&mut self, controls: Controls) {
        let mut payload_operation = None;

        if controls.back_step_pressed {
            payload_operation = Some(OperationType::BackStep)
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

        if let Err(err) = self.bus.write(SIMULATION_WIDGET_NAME.to_string(), msg) {
            error!("failed to publish message: {err}");
        }
    }

    fn handle_incoming_events(&mut self) {
        if let Ok(msg) = self.bus.read(SIMULATION_WIDGET_NAME.to_string()) {
            let step = msg.payload().parse::<usize>().unwrap();
            self.step = Some(step);
            self.info_msg = format!("Step: {step}");
        }
    }
}

impl AppWidget for SimulationProps {
    fn show(&mut self, ui: &mut egui::Ui) {
        let mut controls = Controls::default();

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
    }
}
