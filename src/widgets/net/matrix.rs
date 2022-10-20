use egui::{text::LayoutJob, Align, Color32, FontId, Label, TextFormat};
use ndarray::{ArrayBase, Axis, Ix2, ViewRepr};

use super::graph::MatrixState;
use crate::{netstrat::Bus, widgets::AppWidget};

pub struct Matrix {
    bus: Box<Bus>,
    state: MatrixState,
}

impl Matrix {
    pub fn new(state: MatrixState, bus: Box<Bus>) -> Self {
        Self { bus, state }
    }

    pub fn set_state(&mut self, state: MatrixState) {
        self.state = state;
    }

    // basically index column
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
            let el_string = format!("{}", i);
            if self.state.colored_elements.rows.contains(&i) {
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
            res.push((" \n".to_string(), TextFormat::default()))
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

        // first symbol in col is index
        let idx_string = format!("{}\n", col_idx);
        if self.state.colored_elements.cols.contains(&col_idx) {
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
            let el = m[[col_idx, i]];
            let el_string = format!("{}\n", el);
            if self.state.colored_elements.elements.contains(&(i, col_idx)) {
                res.push((
                    el_string,
                    TextFormat {
                        color: Color32::LIGHT_RED,
                        ..Default::default()
                    },
                ));

                return;
            };

            res.push(match el == 1 {
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
            let filled_column = self.nth_column(&self.state.m.view().reversed_axes(), i);
            cols.push(filled_column);
        });

        ui.columns(n + 1, |ui| {
            for (i, ui) in ui.iter_mut().enumerate() {
                let mut job = LayoutJob::default();
                cols.get(i).unwrap().iter().for_each(|(text, format)| {
                    job.append(text.as_str(), 0.0, format.clone());
                });
                ui.add(Label::new(job).wrap(false));
            }
        });
    }
}
