use crossbeam::channel::{Receiver, Sender};
use egui::{
    text::LayoutJob, Align, Color32, FontId, Label, RichText, ScrollArea, TextEdit, TextFormat,
    TextStyle, Vec2,
};
use ndarray::{Array2, ArrayBase, Axis, Ix2, ViewRepr};

use crate::{
    netstrat::{Bus, Message},
    widgets::AppWidget,
};

pub struct Matrix {
    bus: Box<Bus>,
    m: Array2<u8>,
    selected_rows: Vec<usize>,
    selected_cols: Vec<usize>,
    selected_elements: Vec<[usize; 2]>,
}

impl Matrix {
    pub fn new(m: Array2<u8>, bus: Box<Bus>) -> Self {
        Self {
            m,
            bus,
            selected_rows: Default::default(),
            selected_cols: Default::default(),
            selected_elements: Default::default(),
        }
    }

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
            res.push((
                format!("{}", i),
                TextFormat {
                    font_id: FontId::monospace(9.0),
                    color: Color32::GRAY.linear_multiply(0.1),
                    valign: Align::Center,
                    ..Default::default()
                },
            ));
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

        res.push((
            format!("{}\n", col_idx),
            TextFormat {
                font_id: FontId::monospace(9.0),
                color: Color32::GRAY.linear_multiply(0.1),
                valign: Align::Center,
                ..Default::default()
            },
        ));

        (0..n).for_each(|i| {
            let el = m[[col_idx, i]];
            let el_string = format!("{}\n", el);
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
        let n = self.m.len_of(Axis(0));

        let mut cols = vec![self.first_colum(n)];
        (0..n).for_each(|i| {
            let filled_column = self.nth_column(&self.m.view().reversed_axes(), i);
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
