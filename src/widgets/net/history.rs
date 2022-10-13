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
    current_step: usize,
}

impl History {
    pub fn new() -> Self {
        Self {
            steps: vec![],
            current_step: 0,
        }
    }

    pub fn push(&mut self, step_name: String, data: Data) {
        let step = self.steps.len() + 1;
        self.steps.push(HistoryStep {
            step,
            name: step_name,
            data,
        });
        self.set_current_step(step)
    }

    pub fn get(&mut self, step: usize) -> Option<HistoryStep> {
        let idx = step - 1;
        self.steps.get(idx).cloned()
    }

    pub fn set_current_step(&mut self, step: usize) {
        self.current_step = step
    }

    pub fn get_current_step(&self) -> usize {
        self.current_step
    }

    pub fn iter(&self) -> Iter<'_, HistoryStep> {
        self.steps.iter()
    }
}
