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
    pub has_error: bool,
}

impl LoadingState {
    pub fn new(bounds: &BoundsSet, step: usize, per_page_limit: usize) -> Option<Self> {
        info!("Initializing LoadingState. Bounds: {bounds:?}. Step: {step}. Per page limit: {per_page_limit}.");

        Some(Self {
            pages: Pages::new(bounds.clone(), step, per_page_limit)?,
            ..Default::default()
        })
    }

    pub fn left_edge(&self) -> i64 {
        self.pages.page().0
    }

    pub fn turn_page(&mut self) -> Option<Page> {
        self.pages.next()
    }

    pub fn progress(&mut self) -> f32 {
        if self.pages.len() == 0 {
            return 1.0;
        }

        self.pages.turned_pages as f32 / self.pages.len() as f32
    }
}
