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
    pub pages: Pages,
}

impl LoadingState {
    pub fn new(bounds: &BoundsSet, step: usize, per_page_limit: usize) -> Option<Self> {
        info!("Initializing LoadingState. Bounds: {bounds:?}. Step: {step}. Per page limit: {per_page_limit}.");

        Some(Self {
            pages: Pages::new(bounds.clone(), step, per_page_limit)?,
        })
    }

    pub fn left_edge(&self) -> i64 {
        self.pages.page().0
    }

    pub fn turn_page(&mut self) -> Option<Page> {
        self.pages.next()
    }

    pub fn progress(&mut self) -> f32 {
        self.pages.curr_page_idx as f32 / (self.pages.len() - 1) as f32
    }
}
