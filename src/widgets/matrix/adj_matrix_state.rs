use std::iter::zip;

use ndarray::{Array2, Axis, Ix2};

use super::elements::Elements;

#[derive(Clone, Default)]
pub struct State {
    pub m: Array2<isize>,
    pub colored: Elements,
    pub deleted: Elements,
    pub longest_path: usize,
}

impl State {
    pub fn power(&self, n: usize) -> Self {
        let mut m = self.m.clone();
        (1..n).for_each(|_| {
            m = m.dot(&self.m);
        });

        Self {
            m,
            deleted: self.deleted.clone(),
            longest_path: self.longest_path,
            colored: Elements::default(),
        }
    }

    pub fn uni_matrix(&self) -> Self {
        let n = self.m.len_of(Axis(0));
        let mut m = Array2::zeros((n, n));
        m.diag_mut().iter_mut().for_each(|el| *el = 1);
        Self {
            m,
            longest_path: self.longest_path,
            deleted: self.deleted.clone(),
            colored: Elements::default(),
        }
    }

    pub fn reach_matrix(&self, steps: isize) -> Self {
        let n = self.m.len_of(Axis(0));

        let steps_cnt = match steps == -1 {
            true => self.longest_path,
            false => steps as usize,
        };

        let mut m = Array2::zeros((n, n));
        m.diag_mut().iter_mut().for_each(|el| *el = 1);

        if steps_cnt > 0 {
            (0..steps_cnt).for_each(|i| {
                m += &self.power(i + 1).m;
            });
        }

        Self {
            m: boolean_view(m),
            longest_path: self.longest_path,
            deleted: self.deleted.clone(),
            colored: Elements::default(),
        }
    }

    pub fn cone_distance_matrix(&self, reach_matrix: Array2<isize>) -> Self {
        let n = self.m.len_of(Axis(0));
        let mut m = Array2::zeros((n, n));

        reach_matrix
            .axis_iter(Axis(0))
            .enumerate()
            .for_each(|(i, row_i)| {
                reach_matrix
                    .axis_iter(Axis(0))
                    .enumerate()
                    .for_each(|(j, row_j)| {
                        if i == j {
                            return;
                        }

                        let cell_result =
                            zip(row_i, row_j).map(|(a, b)| (a - b).abs()).sum::<isize>();

                        m[[i, j]] = cell_result;
                        m[[j, i]] = cell_result;
                    });
            });

        Self {
            m,
            longest_path: self.longest_path,
            deleted: self.deleted.clone(),
            colored: Elements::default(),
        }
    }
}

fn boolean_view(mut m: Array2<isize>) -> Array2<isize> {
    m.iter_mut().for_each(|el| {
        if *el > 1 {
            *el = 1
        }
    });
    m
}
