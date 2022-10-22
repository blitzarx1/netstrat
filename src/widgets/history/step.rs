use std::fmt::Display;

use crate::widgets::net_props::State;

#[derive(Clone)]
pub struct Step {
    pub name: String,
    pub data: State,
}

impl Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.name.clone()))
    }
}
