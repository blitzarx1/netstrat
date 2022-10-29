use egui::{text::LayoutJob, Align, Color32, FontId, Grid, Label, TextFormat};
use ndarray::{Array2, ArrayBase, Axis, Ix2, ViewRepr};

use crate::{netstrat::Bus, widgets::AppWidget};

use super::adj_matrix_state::AdjMatrixState;

#[derive(Clone)]
pub struct Matrix {
    adj_state: AdjMatrixState,
    power: usize,
    m_powered: Array2<isize>,
}

impl Matrix {
    pub fn new(state: AdjMatrixState) -> Self {
        Self {
            power: 1,
            adj_state: state.clone(),
            m_powered: state.m,
        }
    }

    pub fn set_state(&mut self, state: AdjMatrixState) {
        self.adj_state.update(state.m, state.colored, state.deleted)
    }

    pub fn set_power(&mut self, power: usize) {
        self.power = power;
        self.m_powered = self.adj_state.power(power).m
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
            if self.adj_state.deleted.rows.contains(&i) {
                return;
            }

            let el_string = format!("{}", i);
            if self.adj_state.colored.rows.contains(&i) && self.power == 1 {
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
        if self.adj_state.colored.cols.contains(&col_idx) && self.power == 1 {
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
            if self.adj_state.deleted.rows.contains(&i) {
                return;
            }

            let el = m[[col_idx, i]];
            let el_string = format!("{}", el);
            if self.adj_state.colored.elements.contains(&(i, col_idx)) && self.power == 1 {
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
        let n = self.adj_state.m.len_of(Axis(0));

        let mut cols = vec![self.first_colum(n)];
        (0..n).for_each(|i| {
            if self.adj_state.deleted.cols.contains(&i) {
                return;
            }

            let mut m = &self.adj_state.m;
            if self.power > 1 {
                m = &self.m_powered;
            }

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
