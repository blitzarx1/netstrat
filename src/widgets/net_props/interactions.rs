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
}

impl Interactions {
    pub fn new(
        selected_cycles: HashSet<usize>,
        graph_settings: NetSettings,
        cone_settings: ConeSettingsInputs,
        selected_history_step: usize,
        nodes_and_edges_settings: NodesAndEdgeSettings,
    ) -> Self {
        Self {
            selected_cycles,
            graph_settings,
            cone_settings,
            selected_history_step,
            nodes_and_edges_settings,
            clicks: Default::default(),
        }
    }
}
