use std::fmt::Display;

use crate::widgets::net::data::Data;

#[derive(Clone)]
pub struct HistoryStep {
    pub name: String,
    pub data: Data,
}

impl Display for HistoryStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.name.clone()))
    }
}
