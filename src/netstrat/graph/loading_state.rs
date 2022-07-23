use chrono::{DateTime, Duration, Utc};
use tracing::info;

use crate::{
    netstrat::bounds::{Bounds, BoundsSet},
    sources::binance::Interval,
};

use super::{
    pages::{Page, Pages},
    props::Props,
};

#[derive(Default, Debug, Clone)]
pub struct LoadingState {
    pub start_time: i64,
    pub pages: Pages,
}

impl LoadingState {
    pub fn new(bounds: &BoundsSet, step: usize, per_page_limit: usize) -> Option<Self> {
        info!("Initializing LoadingState. Bounds: {bounds:?}. Step: {step}. Per page limit: {per_page_limit}.");

        Some(Self {
            start_time: bounds.left_edge()?,
            pages: Pages::new(bounds.clone(), step, per_page_limit)?,
        })
    }

    pub fn turn_page(&self) -> Option<Page> {
        self.clone().pages.next()
    }

    pub fn progress(&self) -> f32 {
        self.pages.curr_page_idx as f32 / self.pages.page_size() as f32 
    }
}
