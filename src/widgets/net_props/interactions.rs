use std::collections::HashSet;

use super::{
    button_clicks::ButtonClicks, cones::ConeSettingsInputs, nodes_and_edges::NodesAndEdgeSettings,
    settings::Settings,
};

pub struct Interactions {
    pub graph_settings: Settings,
    pub cone_settings: ConeSettingsInputs,
    pub clicks: ButtonClicks,
    pub nodes_and_edges_settings: NodesAndEdgeSettings,
    pub selected_cycles: HashSet<usize>,
    pub matrix_power_input: String,
    pub reach_matrix_power_input: String,
}

impl Interactions {
    pub fn new(
        selected_cycles: HashSet<usize>,
        graph_settings: Settings,
        cone_settings: ConeSettingsInputs,
        nodes_and_edges_settings: NodesAndEdgeSettings,
        matrix_power_input: String,
        reach_matrix_power_input: String,
    ) -> Self {
        Self {
            selected_cycles,
            graph_settings,
            cone_settings,
            nodes_and_edges_settings,
            matrix_power_input,
            reach_matrix_power_input,
            clicks: Default::default(),
        }
    }
}
