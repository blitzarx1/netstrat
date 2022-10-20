use ndarray::Array2;

use super::Elements;

#[derive(Clone, Default)]
pub struct State {
    pub m: Array2<u8>,
    pub colored_elements: Elements,
}

impl State {
    pub fn new(m: Array2<u8>, colored_elements: Elements) -> Self {
        Self {
            m,
            colored_elements,
        }
    }
}
