use chrono::{Date, Duration, Utc};

use crate::sources::binance::interval::Interval;

#[derive(Debug, Clone, Copy)]
pub struct Props {
    pub date_start: chrono::Date<Utc>,
    pub date_end: chrono::Date<Utc>,
    pub interval: Interval,
    pub limit: usize,
}

impl Default for Props {
    fn default() -> Self {
        Self {
            date_start: Date::from(Utc::now().date()) - Duration::days(1),
            date_end: Date::from(Utc::now().date()),
            interval: Interval::Minute,
            limit: 1000,
        }
    }
}
