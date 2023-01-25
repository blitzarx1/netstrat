use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct GraphIndex {
    idx: usize,
    time: u128,
}

impl GraphIndex {
    pub fn new(idx: usize) -> GraphIndex {
        GraphIndex {
            idx,
            time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros(),
        }
    }

    pub fn id(&self) -> String {
        format!("{}_{}", self.idx, self.time)
    }
}
