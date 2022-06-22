use std::cmp::Ordering;

use crate::sources::binance::client::Kline;

#[derive(Default, Clone)]
pub struct Data {
    pub vals: Vec<Kline>,
    max_x: f64,
    max_y: f64,
}

impl Data {
    pub fn new(vals: Vec<Kline>) -> Self {
        let max_y = vals
            .iter()
            .max_by(|l, r| {
                if l.close > r.close {
                    return Ordering::Greater;
                }

                Ordering::Less
            })
            .unwrap()
            .close as f64;

        let max_x = vals[vals.len() - 1].t_close as f64;

        Self { vals, max_x, max_y }
    }

    pub fn max_x(&self) -> f64 {
        self.max_x
    }

    pub fn max_y(&self) -> f64 {
        self.max_y
    }
}
