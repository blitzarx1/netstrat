use tracing::{debug, error, info};

use crate::{netstrat::bounds::BoundsSet, sources::binance::Interval};

use super::{loading_state::LoadingState, props::Props};

#[derive(Default, Debug, Clone)]
pub struct State {
    pub loading: LoadingState,
    pub props: Props,
    bounds: BoundsSet,
}

impl State {
    pub fn apply_props(&mut self, props: &Props) {
        debug!("applying new props: {props:?}");

        self.props = props.clone();

        let subtract_res = props.bounds.subtract(&self.bounds);
        if subtract_res.is_none() {
            debug!("found nothing to load");
            self.loading = LoadingState::default();
            return;
        }
        let to_load = subtract_res.unwrap();
        debug!("computed difference to load: {to_load:?}");

        let loading_res = LoadingState::new(&to_load, State::step(props.interval), props.limit);
        if loading_res.is_none() {
            error!("failed to initialize loading state");
            return;
        }
        let loading = loading_res.unwrap();
        debug!("initialized loading state: {loading:?}");

        let new_bounds = self.bounds.merge(&props.bounds);
        debug!("computed new_bounds: {new_bounds:?}");

        self.loading = loading;
        self.bounds = new_bounds;
        self.props = props.clone();
    }

    fn step(i: Interval) -> usize {
        match i {
            Interval::Minute => 60 * 1000,
            Interval::Hour => 60 * 60 * 1000,
            Interval::Day => 60 * 60 * 24 * 1000,
        }
    }
}
