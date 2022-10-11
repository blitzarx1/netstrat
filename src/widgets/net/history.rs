use std::slice::Iter;

use super::data::Data;

#[derive(Clone)]
pub struct HistoryStep {
    pub step: usize,
    pub name: String,
    pub data: Data,
}
pub struct History {
    steps: Vec<HistoryStep>,
}

impl History {
    pub fn new() -> Self {
        Self { steps: vec![] }
    }

    pub fn push(&mut self, step_name: String, data: Data) {
        let step = self.steps.len() + 1;
        self.steps.push(HistoryStep {
            step,
            name: step_name,
            data,
        });
    }

    pub fn get_and_crop(&mut self, step: usize) -> HistoryStep {
        let idx = step - 1;
        let history_step = self.steps.get(idx).unwrap().clone();

        self.steps = self.steps[0..idx + 1].to_vec();

        history_step
    }

    pub fn iter(&self) -> Iter<'_, HistoryStep> {
        self.steps.iter()
    }
}
