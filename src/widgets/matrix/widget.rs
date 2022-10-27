use egui::{
    text::LayoutJob, Align, CentralPanel, Color32, FontId, Grid, Label, RichText, ScrollArea,
    TextBuffer, TextEdit, TextFormat, Vec2,
};
use egui_extras::StripBuilder;
use ndarray::{Array2, ArrayBase, Axis, Ix2, ViewRepr};

use crate::{netstrat::Bus, widgets::AppWidget};

use super::state::State;

#[derive(Clone)]
pub struct Matrix {
    bus: Box<Bus>,
    state: State,
    power: usize,
    m_powered: Array2<u8>,
}

impl Matrix {
    pub fn new(state: State, bus: Box<Bus>) -> Self {
        Self {
            bus,
            power: 1,
            state: state.clone(),
            m_powered: state.m,
        }
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }

    pub fn set_power(&mut self, power: usize) {
        self.power = power;
        if power > 1 {
            self.m_powered = self.state.m.clone();
            (1..power).for_each(|_| {
                self.m_powered = self.m_powered.dot(&self.m_powered);
            });
        }
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
            if self.state.colored.rows.contains(&i) && self.power == 1 {
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
        m: &ArrayBase<ViewRepr<&u8>, Ix2>,
        col_idx: usize,
    ) -> Vec<(String, TextFormat)> {
        let n = m.len_of(Axis(0));
        let mut res = Vec::with_capacity(n + 1);

        // first symbol in col is col index
        let idx_string = format!("{}", col_idx);
        if self.state.colored.cols.contains(&col_idx) && self.power == 1 {
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
            if self.state.colored.elements.contains(&(i, col_idx)) && self.power == 1 {
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

            let mut m = &self.state.m;
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
