use std::cmp::Ordering;

use chrono::{DateTime, NaiveDateTime, Utc};
use egui::Color32;
use tracing::{debug, trace};

use crate::sources::binance::Kline;

#[derive(Default, Clone)]
pub struct Data {
    pub vals: Vec<Kline>,
    max_x: f64,
    min_x: f64,
    max_y: f64,
    min_y: f64,
    max_vol: f64,
}

impl Data {
    pub fn new_candle() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn append(&mut self, vals: &mut Vec<Kline>) {
        self.vals.append(vals);
        self.compute_stats();
    }

    pub fn max_x(&self) -> f64 {
        self.max_x
    }

    pub fn max_y(&self) -> f64 {
        self.max_y
    }

    pub fn min_y(&self) -> f64 {
        self.min_y
    }

    pub fn min_x(&self) -> f64 {
        self.min_x
    }

    pub fn max_vol(&self) -> f64 {
        self.max_vol
    }

    pub fn format_ts(ts: f64) -> String {
        let secs = (ts / 1000f64) as i64;
        let naive = NaiveDateTime::from_timestamp(secs, 0);
        let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);

        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub fn k_color(k: &Kline) -> Color32 {
        match k.open > k.close {
            true => Color32::LIGHT_RED,
            false => Color32::LIGHT_GREEN,
        }
    }

    fn compute_stats(&mut self) {
        self.vals.sort_by_key(|el| el.t_close);

        self.max_y = self
            .vals
            .iter()
            .max_by(|l, r| {
                if l.high > r.high {
                    return Ordering::Greater;
                }

                Ordering::Less
            })
            .unwrap()
            .high as f64;

        self.min_y = self
            .vals
            .iter()
            .min_by(|l, r| {
                if l.low < r.low {
                    return Ordering::Less;
                }

                Ordering::Greater
            })
            .unwrap()
            .low as f64;

        self.max_x = self.vals.last().unwrap().t_close as f64;
        self.min_x = self.vals.first().unwrap().t_open as f64;

        debug!(
            "computed data props : max_x: {}, min_x: {}, max_y: {}, min_y: {}",
            self.max_x, self.min_x, self.max_y, self.min_y,
        );
    }
}
