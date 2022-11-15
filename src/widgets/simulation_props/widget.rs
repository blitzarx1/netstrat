use tracing::error;

use crate::{
    netstrat::{Bus, Message},
    widgets::AppWidget,
};

use super::Controls;

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

        if controls.next_step_pressed {
            if let Err(err) = self.bus.write(
                SIMULATION_WIDGET_NAME.to_string(),
                Message::new("".to_string()),
            ) {
                error!("failed to publish message: {err}");
            }
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
            if self.step.is_some() {
                ui.add_space(5.0);
                ui.label(format!("step: {:?}", self.step.unwrap()));
            };
        });

        self.update(controls);
    }
}
