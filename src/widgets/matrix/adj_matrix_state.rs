use eframe::epaint::ahash::{HashMap, HashMapExt};
use ndarray::Array2;
use tracing::debug;

use super::elements::Elements;
const MAX_CASH_LENGTH: usize = 10;

#[derive(Clone, Default)]
pub struct AdjMatrixState {
    pub m: Array2<isize>,
    pub colored: Elements,
    pub deleted: Elements,
    last_powers: HashMap<usize, Self>,
}

impl AdjMatrixState {
    pub fn new(m: Array2<isize>, colored: Elements, deleted: Elements) -> Self {
        Self {
            m,
            colored,
            deleted,
            last_powers: HashMap::with_capacity(MAX_CASH_LENGTH),
        }
    }

    pub fn update(&mut self, m: Array2<isize>, colored: Elements, deleted: Elements) {
        self.m = m;
        self.colored = colored;
        self.deleted = deleted;
    }

    fn store_computed_power(&mut self, n: usize, computed_power: Self) {
        if self.last_powers.len() > MAX_CASH_LENGTH {
            debug!("cash reached max size; trimming");
            let first_key = *self.last_powers.keys().next().unwrap();
            self.last_powers.remove(&first_key);
        }

        self.last_powers.insert(n, computed_power);
    }

    pub fn power(&mut self, n: usize) -> Self {
        if let Some(computed_power) = self.last_powers.get(&n) {
            debug!("got adj matrix power from cash");
            return computed_power.clone();
        }

        debug!("computing adj matrix power");
        match n > 1 {
            false => self.clone(),
            true => {
                let mut m = self.m.clone();
                (1..n).for_each(|_| {
                    m = m.dot(&self.m);
                });

                let res = AdjMatrixState::new(m, Elements::default(), Elements::default());

                self.store_computed_power(n, res.clone());

                res
            }
        }
    }
}
