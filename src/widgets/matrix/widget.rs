use std::{collections::HashMap, default};

use egui::{text::LayoutJob, Align, Color32, FontId, Grid, Label, TextFormat};
use ndarray::{Array2, ArrayBase, Axis, Ix2, ViewRepr};
use tracing::debug;

use crate::widgets::AppWidget;

const MAX_CASH_LENGTH: usize = 10;

use super::adj_matrix_state::State;

#[derive(Clone)]
pub struct Matrix {
    state: State,
    last_powers: HashMap<usize, State>,
    last_reach: HashMap<isize, State>,
}

impl Matrix {
    pub fn new(state: State) -> Self {
        let mut last_powers = HashMap::with_capacity(MAX_CASH_LENGTH);
        let mut last_reach = HashMap::with_capacity(MAX_CASH_LENGTH);
        last_powers.insert(1, state.clone());
        last_reach.insert(1, state.reach_matrix(1));
        Self {
            state,
            last_powers,
            last_reach,
        }
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
        self.last_powers = HashMap::with_capacity(MAX_CASH_LENGTH);
        self.last_reach = HashMap::with_capacity(MAX_CASH_LENGTH);
    }

    pub fn powered(&mut self, n: usize) -> Self {
        Self {
            state: self.get_power(n),
            last_powers: Default::default(),
            last_reach: Default::default(),
        }
    }

    pub fn reach(&mut self, steps: isize) -> Self {
        if let Some(computed_reach) = self.last_reach.get(&steps) {
            debug!("got reach matrix power from cash");
            return Self {
                state: computed_reach.clone(),
                last_powers: Default::default(),
                last_reach: Default::default(),
            };
        }

        debug!("computing reach matrix power");

        let res = self.state.reach_matrix(steps);
        self.store_computed_reach(steps, res.clone());

        Self {
            state: res,
            last_powers: Default::default(),
            last_reach: Default::default(),
        }
    }

    pub fn cone_distance(&mut self) -> Self {
        let reach = self.reach(-1);

        let res = self.state.cone_distance_matrix(reach.state.m);
        Self {
            state: res,
            last_powers: Default::default(),
            last_reach: Default::default(),
        }
    }

    fn get_power(&mut self, n: usize) -> State {
        if let Some(computed_power) = self.last_powers.get(&n) {
            debug!("got adj matrix power from cash");
            return computed_power.clone();
        }

        debug!("computing adj matrix power");

        let res = match n {
            0 => self.state.uni_matrix(),
            _ => self.state.power(n),
        };

        self.store_computed_power(n, res.clone());
        res
    }

    fn store_computed_power(&mut self, n: usize, computed_power: State) {
        if self.last_powers.len() > MAX_CASH_LENGTH {
            debug!("cash reached max size; trimming");
            let first_key = *self.last_powers.keys().next().unwrap();
            self.last_powers.remove(&first_key);
        }

        self.last_powers.insert(n, computed_power);
    }

    fn store_computed_reach(&mut self, n: isize, computed_reach: State) {
        if self.last_reach.len() > MAX_CASH_LENGTH {
            debug!("cash reached max size; trimming");
            let first_key = *self.last_reach.keys().next().unwrap();
            self.last_reach.remove(&first_key);
        }

        self.last_reach.insert(n, computed_reach);
    }

    // row index column
    fn first_colum(&self, n: usize) -> Vec<(String, TextFormat)> {
        let mut res = vec![(
            "\n".to_string(),
            TextFormat {
                font_id: FontId::monospace(9.0),
                color: Color32::GRAY.linear_multiply(0.1),
                valign: Align::Center,
                ..Default::default()
            },
        )];
        (0..n).for_each(|i| {
            if self.state.deleted.rows.contains(&i) {
                return;
            }

            let el_string = format!("{}", i);
            if self.state.colored.rows.contains(&i) {
                res.push((
                    el_string,
                    TextFormat {
                        font_id: FontId::monospace(9.0),
                        color: Color32::LIGHT_RED,
                        valign: Align::Center,
                        ..Default::default()
                    },
                ));
            } else {
                res.push((
                    el_string,
                    TextFormat {
                        font_id: FontId::monospace(9.0),
                        color: Color32::GRAY.linear_multiply(0.1),
                        valign: Align::Center,
                        ..Default::default()
                    },
                ));
            }
        });
        res
    }

    fn nth_column(
        &self,
        m: &ArrayBase<ViewRepr<&isize>, Ix2>,
        col_idx: usize,
    ) -> Vec<(String, TextFormat)> {
        let n = m.len_of(Axis(0));
        let mut res = Vec::with_capacity(n + 1);

        // first symbol in col is col index
        let idx_string = format!("{}", col_idx);
        if self.state.colored.cols.contains(&col_idx) {
            res.push((
                idx_string,
                TextFormat {
                    font_id: FontId::monospace(9.0),
                    color: Color32::LIGHT_RED,
                    valign: Align::Center,
                    ..Default::default()
                },
            ));
        } else {
            res.push((
                idx_string,
                TextFormat {
                    font_id: FontId::monospace(9.0),
                    color: Color32::GRAY.linear_multiply(0.1),
                    valign: Align::Center,
                    ..Default::default()
                },
            ));
        }

        (0..n).for_each(|i| {
            if self.state.deleted.rows.contains(&i) {
                return;
            }

            let el = m[[col_idx, i]];
            let el_string = format!("{}", el);
            if self.state.colored.elements.contains(&(i, col_idx)) {
                res.push((
                    el_string,
                    TextFormat {
                        color: Color32::LIGHT_RED,
                        ..Default::default()
                    },
                ));

                return;
            };

            res.push(match el != 0 {
                true => (
                    el_string,
                    TextFormat {
                        color: Color32::WHITE,
                        ..Default::default()
                    },
                ),
                false => (
                    el_string,
                    TextFormat {
                        color: Color32::GRAY.linear_multiply(0.5),
                        ..Default::default()
                    },
                ),
            })
        });

        res
    }
}

impl AppWidget for Matrix {
    fn show(&mut self, ui: &mut egui::Ui) {
        let n = self.state.m.len_of(Axis(0));

        let mut cols = vec![self.first_colum(n)];
        (0..n).for_each(|i| {
            if self.state.deleted.cols.contains(&i) {
                return;
            }

            let m = &self.state.m;

            let filled_column = self.nth_column(&m.view().reversed_axes(), i);
            cols.push(filled_column);
        });

        let cols_num = cols.len();
        Grid::new("mat")
            .min_col_width(14.0)
            .max_col_width(14.0)
            .striped(true)
            .show(ui, |ui| {
                (0..cols_num).for_each(|row| {
                    (0..cols_num).for_each(|col| {
                        let mut job = LayoutJob::default();
                        let (text, format) = cols.get(col).unwrap().get(row).unwrap();
                        job.append(text.as_str(), 0.0, format.clone());
                        ui.add(Label::new(job).wrap(false));
                    });
                    ui.end_row();
                });
            });
    }
}
