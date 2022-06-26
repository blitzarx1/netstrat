use chrono::{DateTime, Duration, Utc};
use tracing::debug;

use crate::sources::binance::interval::Interval;

use super::props::Props;

static MAX_LIMIT: u32 = 1000;

#[derive(Default, Debug, Clone, Copy)]
pub struct LoadingState {
    pub triggered: bool,
    pub initial: Props,
    pub received: u32,
    pub pages: u32,
    pub last_page_limit: usize,
}

impl LoadingState {
    pub fn from_graph_props(props: &Props) -> Self {
        let diff_days = props.date_end - props.date_start;

        // debug!("props: start{:?}, end: {:?}", props.date_start, props.date_end);

        match props.interval {
            Interval::Minute => {
                let pages_proto = Duration::num_minutes(&diff_days) as f32 / MAX_LIMIT as f32;
                let pages = pages_proto.ceil() as u32;
                let last_page_limit = (pages_proto.fract() * MAX_LIMIT as f32) as usize;

                LoadingState {
                    triggered: false,
                    initial: props.clone(),
                    pages,
                    received: 0,
                    last_page_limit,
                }
            }
            Interval::Hour => {
                let pages_proto = Duration::num_hours(&diff_days) as f32 / MAX_LIMIT as f32;
                let pages = pages_proto.ceil() as u32;
                let last_page_limit = (pages_proto.fract() * MAX_LIMIT as f32) as usize;

                LoadingState {
                    triggered: false,
                    initial: props.clone(),
                    pages,
                    received: 0,
                    last_page_limit,
                }
            }
            Interval::Day => {
                let pages_proto = Duration::num_days(&diff_days) as f32 / MAX_LIMIT as f32;
                let pages = pages_proto.ceil() as u32;
                let last_page_limit = (pages_proto.fract() * MAX_LIMIT as f32) as usize;

                LoadingState {
                    triggered: false,
                    initial: props.clone(),
                    pages,
                    received: 0,
                    last_page_limit,
                }
            }
        }
    }

    pub fn left_edge(&self) -> DateTime<Utc> {
        let covered: Duration;

        match self.initial.interval {
            Interval::Minute => {
                covered = Duration::minutes((self.received * self.initial.limit as u32) as i64)
            }
            Interval::Hour => {
                covered = Duration::hours((self.received * self.initial.limit as u32) as i64)
            }
            Interval::Day => {
                covered = Duration::days((self.received * self.initial.limit as u32) as i64)
            }
        };

        self.initial.date_start.and_hms(0, 0, 0) + covered
    }

    pub fn inc_received(&mut self) {
        self.received += 1;
    }

    pub fn is_finished(&self) -> bool {
        return self.progress() == 1f32;
    }

    pub fn progress(&self) -> f32 {
        if self.pages == 0 {
            return 1f32;
        }
        self.received as f32 / self.pages as f32
    }

    pub fn is_last_page(&self) -> bool {
        return self.pages - self.received == 1;
    }
}
