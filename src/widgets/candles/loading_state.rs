use tracing::debug;

use super::{
    pages::{Page, Pages}, bounds::BoundsSet,
};

#[derive(Default, Debug, Clone)]
pub struct LoadingState {
    loaded_pages: usize,
    pages: Pages,
    curr_page: Page,
    pub has_error: bool,
}

impl LoadingState {
    pub fn new(bounds: &BoundsSet, step: usize, per_page_limit: usize) -> Option<Self> {
        debug!("initializing LoadingState: bounds: {bounds:?}; step: {step}; per page limit: {per_page_limit}");

        Some(Self {
            pages: Pages::new(bounds.clone(), step, per_page_limit)?,
            ..Default::default()
        })
    }

    pub fn left_edge(&self) -> i64 {
        self.curr_page.0
    }

    pub fn get_next_page(&mut self) -> Option<Page> {
        let res = self.pages.next();
        if let Some(p) = res {
            self.curr_page = p.clone();
            return Some(p);
        };

        None
    }

    pub fn inc_loaded_pages(&mut self, cnt: usize) {
        self.loaded_pages += cnt;
    }

    pub fn progress(&mut self) -> f32 {
        if self.pages.len() == 0 {
            return 1.0;
        }

        self.loaded_pages as f32 / self.pages.len() as f32
    }

    pub fn page_size(&self) -> usize {
        self.pages.page_size(self.curr_page.clone())
    }

    pub fn pages(&self) -> usize {
        self.pages.len()
    }
}
