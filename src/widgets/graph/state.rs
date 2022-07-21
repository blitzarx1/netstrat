use chrono::{DateTime, Duration, Utc};
use tracing::info;

use crate::{netstrat::bounds::BoundsSet, sources::binance::Interval};

use super::props::Props;

#[derive(Default, Debug, Clone)]
pub struct State {
    pub triggered: bool,
    pub props: Props,
    pub received: u32,
    pub pages: u32,
    pub last_page_limit: usize,
    bounds: BoundsSet,
}

impl State {
    pub fn from_graph_props(props: &Props) -> Self {
        info!("got props: {props:?}");

        let diff = props.end_time() - props.start_time();

        info!("loading graph for duration: {diff:?}");

        let state: State;

        match props.interval {
            Interval::Minute => {
                let pages_proto = Duration::num_minutes(&diff) as f32 / props.limit as f32;
                let pages = pages_proto.ceil() as u32;
                let last_page_limit = (pages_proto.fract() * props.limit as f32) as usize;

                state = State {
                    triggered: false,
                    props: props.clone(),
                    pages,
                    received: 0,
                    last_page_limit,
                    bounds: BoundsSet::new(vec![]),
                };
            }
            Interval::Hour => {
                let pages_proto = Duration::num_hours(&diff) as f32 / props.limit as f32;
                let pages = pages_proto.ceil() as u32;
                let last_page_limit = (pages_proto.fract() * props.limit as f32) as usize;

                state = State {
                    triggered: false,
                    props: props.clone(),
                    pages,
                    received: 0,
                    last_page_limit,
                    bounds: BoundsSet::new(vec![]),
                };
            }
            Interval::Day => {
                let pages_proto = Duration::num_days(&diff) as f32 / props.limit as f32;
                let pages = pages_proto.ceil() as u32;
                let last_page_limit = (pages_proto.fract() * props.limit as f32) as usize;

                state = State {
                    triggered: false,
                    props: props.clone(),
                    pages,
                    received: 0,
                    last_page_limit,
                    bounds: BoundsSet::new(vec![]),
                };
            }
        };

        info!("created state for total duration {diff}: {state:?}");

        state
    }

    pub fn left_edge(&self) -> DateTime<Utc> {
        let covered: Duration;

        match self.props.interval {
            Interval::Minute => {
                covered = Duration::minutes((self.received * self.props.limit as u32) as i64)
            }
            Interval::Hour => {
                covered = Duration::hours((self.received * self.props.limit as u32) as i64)
            }
            Interval::Day => {
                covered = Duration::days((self.received * self.props.limit as u32) as i64)
            }
        };

        self.props.date_start.and_hms(0, 0, 0) + covered
    }

    pub fn inc_received(&mut self) {
        self.received += 1;
    }

    // TODO:
    // pub fn register_received_data(&mut self, b: BoundsSequence) -> BoundsSequence{
    //     self.bounds.add_with_diff(b)
    // }

    pub fn is_finished(&self) -> bool {
        return self.triggered && self.progress() == 1f32;
    }

    pub fn in_progress(&self) -> bool {
        return self.triggered && self.progress() != 1f32;
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
