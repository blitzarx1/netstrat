use std::fmt::Display;

use super::step_difference::StepDifference;

#[derive(Clone)]
pub struct Step {
    pub name: String,
    /// parent_difference has none value only for root node
    pub parent_difference: StepDifference,
}

impl Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.name.clone()))
    }
}
