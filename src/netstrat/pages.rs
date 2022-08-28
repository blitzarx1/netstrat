use tracing::{debug, error, info};

use crate::netstrat::bounds::BoundsSet;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Page(pub i64, pub i64);

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Pages {
    curr_page_idx: usize,
    vals: Vec<Page>,
    step: usize,
}

impl Pages {
    /// Creates a new Pages instance.
    ///
    /// Page is a pair of start and end
    /// where the start is included in the range and the end is not.
    pub fn new(bounds: BoundsSet, step: usize, limit: usize) -> Option<Self> {
        info!("Initializing Pages. Bounds: {bounds:?}. Step: {step}. Limit: {limit}.");

        if step < 1 {
            error!("Invalid step. Step must be greater than 0.");
            return None;
        }

        let mut vals = vec![];
        bounds.vals().iter_mut().for_each(|b| {
            if b.len() <= limit*step as usize  {
                debug!("Not iterating inside bounds due to its size being less than limit. Taking it to page as a whole. Bounds: {b:?}. Step: {step}.");
                vals.push(Page(b.0, b.1));
                return ;
            }

            debug!("Iterating inside bounds constructing pages. Bounds: {b:?}. Step: {step}.");

            let mut page_start = b.0;
            loop {
                let mut page_end = page_start + (step*limit) as i64;
                if page_end > b.1 {
                    page_end = b.1;
                }

                vals.push(Page(page_start, page_end));
                if page_end == b.1 {
                    break;
                }
                page_start = page_end;
            }
        });

        info!("Computed pages: {vals:?}.");

        Some(Self {
            vals,
            step,
            ..Default::default()
        })
    }

    pub fn len(&self) -> usize {
        self.vals.len()
    }

    pub fn next(&mut self) -> Option<Page> {
        if let Some(page) = self.vals.get(self.curr_page_idx) {
            self.curr_page_idx += 1;
            return Some(page.clone());
        }

        None
    }

    pub fn page_size(&self, page: Page) -> usize {
        ((page.1 - page.0) / self.step as i64) as usize
    }
}

#[cfg(test)]
mod pages_tests {
    use crate::netstrat::bounds::Bounds;

    use super::*;

    #[test]
    fn test_pages_new() {
        let pages_res = Pages::new(BoundsSet::new(vec![Bounds(0, 50), Bounds(60, 150)]), 1, 50);
        assert_ne!(pages_res, None);
        assert_eq!(
            pages_res.unwrap(),
            Pages {
                vals: vec![Page(0, 50), Page(60, 110), Page(110, 150)],
                step: 1,
                ..Default::default()
            }
        );

        let pages_res = Pages::new(BoundsSet::new(vec![Bounds(0, 50), Bounds(60, 150)]), 2, 25);
        assert_ne!(pages_res, None);
        assert_eq!(
            pages_res.unwrap(),
            Pages {
                vals: vec![Page(0, 50), Page(60, 110), Page(110, 150)],
                step: 2,
                ..Default::default()
            }
        );
    }
}
