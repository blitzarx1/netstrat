#[derive(PartialEq, Eq, Clone, Debug)]
pub enum EdgeWeight {
    /// Fixed weight
    Fixed,
    /// Random weigh in range 0..1
    Random,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Settings {
    pub ini_cnt: usize,
    pub fin_cnt: usize,
    pub total_cnt: usize,
    pub no_twin_edges: bool,
    pub max_out_degree: usize,
    pub edge_weight_type: EdgeWeight,
    pub edge_weight: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ini_cnt: 5,
            fin_cnt: 5,
            total_cnt: 20,
            max_out_degree: 3,
            no_twin_edges: true,
            edge_weight_type: EdgeWeight::Fixed,
            edge_weight: 1.0,
        }
    }
}