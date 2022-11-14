use tracing::error;

use crate::{
    netstrat::{Bus, Message},
    widgets::AppWidget,
};

use super::Controls;

const WIDGET_NAME: &str = "simulator";

pub struct Simulator {
    bus: Bus,
}

impl Simulator {
    pub fn new(bus: Bus) -> Self {
        Self { bus }
    }

    fn update(&mut self, controls: Controls) {
        if controls.next_step_pressed {
            if let Err(err) = self
                .bus
                .write(WIDGET_NAME.to_string(), Message::new("pressed".to_string()))
            {
                error!("failed to publish message: {err}");
            }
        }
    }
}

impl AppWidget for Simulator {
    fn show(&mut self, ui: &mut egui::Ui) {
        let mut controls = Controls::default();

        if ui.button("next step").clicked() {
            controls.next_step_pressed = true
        }

        self.update(controls);
    }
}
