use ndarray::Array2;

use super::elements::Elements;

#[derive(Clone, Default)]
pub struct State {
    pub m: Array2<u8>,
    pub colored: Elements,
    pub deleted: Elements,
}

impl State {
    pub fn new(m: Array2<u8>, colored: Elements, deleted: Elements) -> Self {
        Self {
            m,
            colored,
            deleted,
        }
    }
}
