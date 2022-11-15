#[derive(Default, Clone)]
pub struct SimulationState {
    step: Option<usize>,
}

impl SimulationState {
    pub fn step(&self) -> Option<usize> {
        self.step
    }

    pub fn reset(&mut self) {
        self.step = None
    }

    pub fn inc(&mut self) {
        if let Some(step) = self.step {
            self.step = Some(step + 1);
            return;
        }

        self.step = Some(0);
    }
}
