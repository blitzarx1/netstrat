use tracing::error;

use crate::{
    netstrat::{Bus, Message},
    widgets::AppWidget,
};

use super::Controls;

pub const SIMULATION_WIDGET_NAME: &str = "simulation";

pub struct SimulationProps {
    bus: Bus,
}

impl SimulationProps {
    pub fn new(bus: Bus) -> Self {
        Self { bus }
    }

    fn update(&mut self, controls: Controls) {
        if controls.next_step_pressed {
            // TODO: get payload from message serialization
            let payload = "pressed";
            if let Err(err) = self
                .bus
                .write(SIMULATION_WIDGET_NAME.to_string(), Message::new(payload.to_string()))
            {
                error!("failed to publish message: {err}");
            }
        }
    }
}

impl AppWidget for SimulationProps {
    fn show(&mut self, ui: &mut egui::Ui) {
        let mut controls = Controls::default();

        if ui.button("▶").clicked() {
            controls.next_step_pressed = true
        }

        self.update(controls);
    }
}
