use std::collections::HashSet;

use super::{
    button_clicks::ButtonClicks, cones::ConeSettingsInputs, nodes_and_edges::NodesAndEdgeSettings,
    settings::NetSettings,
};

pub struct Interactions {
    pub graph_settings: NetSettings,
    pub cone_settings: ConeSettingsInputs,
    pub clicks: ButtonClicks,
    pub nodes_and_edges_settings: NodesAndEdgeSettings,
    pub selected_cycles: HashSet<usize>,
    pub selected_history_step: usize,
    pub matrix_power_input: String,
}

impl Interactions {
    pub fn new(
        selected_cycles: HashSet<usize>,
        graph_settings: NetSettings,
        cone_settings: ConeSettingsInputs,
        selected_history_step: usize,
        nodes_and_edges_settings: NodesAndEdgeSettings,
        matrix_power_input: String,
    ) -> Self {
        Self {
            selected_cycles,
            graph_settings,
            cone_settings,
            selected_history_step,
            nodes_and_edges_settings,
            matrix_power_input,
            clicks: Default::default(),
        }
    }
}
