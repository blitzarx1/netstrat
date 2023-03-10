use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::step_difference::StepDifference;

#[derive(Clone, Serialize, Deserialize)]
pub struct Step {
    pub name: String,
    pub parent_difference: StepDifference,
}

impl Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.name.clone()))
    }
}
