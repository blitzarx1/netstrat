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

pub struct SimulationProps {
    bus: Bus,
    step: Option<usize>,
}

impl SimulationProps {
    pub fn new(bus: Bus) -> Self {
        Self {
            bus,
            step: Default::default(),
        }
    }

    fn update(&mut self, controls: Controls) {
        self.check_events();

        let mut payload_operation = None;

        if controls.next_step_pressed {
            payload_operation = Some(OperationType::NextStep)
        }

        if controls.reset_pressed {
            payload_operation = Some(OperationType::Reset)
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

    fn check_events(&mut self) {
        if let Ok(msg) = self.bus.read(SIMULATION_WIDGET_NAME.to_string()) {
            self.step = Some(msg.payload().parse::<usize>().unwrap())
        }
    }
}

impl AppWidget for SimulationProps {
    fn show(&mut self, ui: &mut egui::Ui) {
        let mut controls = Controls::default();

        ui.horizontal_top(|ui| {
            if ui.button("▶").clicked() {
                controls.next_step_pressed = true
            };
            ui.add_space(5.0);
            if ui.button("⟲").clicked() {
                controls.reset_pressed = true
            }
        });

        ui.separator();

        if self.step.is_some() {
            ui.label(format!("Step: {:?}", self.step.unwrap()));
        } else {
            ui.label("Press ▶ to start simulation");
        };

        self.update(controls);
    }
}
