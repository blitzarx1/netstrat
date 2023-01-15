use petgraph::Direction;
use serde::{Deserialize, Serialize};
use Direction::Outgoing;

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub enum EdgeWeight {
    /// Fixed weight
    Fixed,
    /// Random weight in range 0..1
    Random,
}

impl Default for EdgeWeight {
    fn default() -> Self {
        EdgeWeight::Fixed
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Settings {
    pub ini_cnt: usize,
    pub fin_cnt: usize,
    pub total_cnt: usize,
    pub diamond_filter: bool,
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
            diamond_filter: true,
            edge_weight_type: EdgeWeight::Fixed,
            edge_weight: 1.0,
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct ConeSettings {
    pub roots_names: Vec<String>,
    pub dir: Direction,
    pub max_steps: i32,
}

impl Default for ConeSettings {
    fn default() -> Self {
        Self {
            roots_names: Default::default(),
            dir: Outgoing,
            max_steps: -1,
        }
    }
}
