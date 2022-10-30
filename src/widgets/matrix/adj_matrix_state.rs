use ndarray::{Array2, Axis, Ix2};

use super::elements::Elements;

#[derive(Clone, Default)]
pub struct State {
    pub m: Array2<isize>,
    pub colored: Elements,
    pub deleted: Elements,
}

impl State {
    pub fn new(m: Array2<isize>, colored: Elements, deleted: Elements) -> Self {
        Self {
            m,
            colored,
            deleted,
        }
    }

    pub fn power(&self, n: usize) -> Self {
        let mut m = self.m.clone();
        (1..n).for_each(|_| {
            m = m.dot(&self.m);
        });

        State::new(m, Elements::default(), self.deleted.clone())
    }

    pub fn uni_matrix(&self) -> Self {
        let n = self.m.len_of(Axis(0));
        let mut m = Array2::zeros((n, n));
        m.diag_mut().iter_mut().for_each(|el| *el = 1);
        Self {
            m,
            colored: Elements::default(),
            deleted: self.deleted.clone(),
        }
    }
}
