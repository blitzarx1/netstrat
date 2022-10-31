#[derive(Default, Clone)]
pub struct ButtonClicks {
    pub reset: bool,
    pub create: bool,
    pub color_cones: bool,
    pub color_cycles: bool,
    pub export_dot: bool,
    pub export_svg: bool,
    pub delete_cone: bool,
    pub delete_cycles: bool,
    pub color_nodes_and_edges: bool,
    pub delete_nodes_and_edges: bool,
    pub open_dot_preview: bool,
    pub apply_power: bool,
    pub apply_reach_power: bool,
}
