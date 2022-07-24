use std::cmp::Ordering;

use chrono::{DateTime, NaiveDateTime, Utc};
use egui::Color32;
use tracing::debug;

use crate::sources::binance::Kline;

#[derive(Default, Clone)]
pub struct Data {
    pub vals: Vec<Kline>,
    max_x: f64,
    max_y: f64,
    min_y: f64,
    max_vol: f64,
}

impl Data {
    pub fn new(vals: Vec<Kline>) -> Self {
        let max_y = vals
            .iter()
            .max_by(|l, r| {
                if l.high > r.high {
                    return Ordering::Greater;
                }

                Ordering::Less
            })
            .unwrap()
            .high as f64;

        let min_y = vals
            .iter()
            .min_by(|l, r| {
                if l.low < r.low {
                    return Ordering::Less;
                }

                Ordering::Greater
            })
            .unwrap()
            .low as f64;

        let max_vol = vals
            .iter()
            .max_by(|l, r| {
                if l.volume > r.volume {
                    return Ordering::Greater;
                }

                Ordering::Less
            })
            .unwrap()
            .volume as f64;

        let max_x = vals[vals.len() - 1].t_close as f64;

        debug!(
            "Computed data props: max_x: {max_x},  max_y: {max_y}, min_y: {min_y}, max_vol: {max_vol}."
        );

        Self {
            vals,
            max_x,
            max_y,
            min_y,
            max_vol,
        }
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
}
