use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::widgets::net_props::Elements;

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Difference {
    pub plus: Elements,
    pub minus: Elements,
}

impl Display for Difference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("+: {}\n-: {}", self.plus, self.minus))
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct StepDifference {
    pub elements: Option<Difference>,
    pub colored: Option<Difference>,
    pub signal_holders: Option<Difference>,
}

impl StepDifference {
    pub fn squash(&self, other: &StepDifference) -> StepDifference {
        let mut elements_plus = Elements::default();
        let mut elements_minus = Elements::default();
        if let Some(new_elements) = other.elements.clone() {
            elements_plus = new_elements.plus;
            elements_minus = new_elements.minus;

            if let Some(self_elements) = self.elements.clone() {
                elements_plus = elements_plus.union(&self_elements.plus);
                elements_minus = elements_minus.union(&self_elements.minus);
            }
        }

        let mut colored_plus = Elements::default();
        let mut colored_minus = Elements::default();
        if let Some(new_colored) = other.colored.clone() {
            colored_plus = new_colored.plus;
            colored_minus = new_colored.minus;

            if let Some(self_colored) = self.colored.clone() {
                colored_plus = colored_plus.union(&self_colored.plus);
                colored_minus = colored_minus.union(&self_colored.minus);
            }
        }

        let mut signal_holders_plus = Elements::default();
        let mut signal_holders_minus = Elements::default();
        if let Some(new_signal_holders) = other.signal_holders.clone() {
            signal_holders_plus = new_signal_holders.plus;
            signal_holders_minus = new_signal_holders.minus;

            if let Some(self_signal_holders) = self.signal_holders.clone() {
                signal_holders_plus = signal_holders_plus.union(&self_signal_holders.plus);
                signal_holders_minus = signal_holders_minus.union(&self_signal_holders.minus);
            }
        }

        StepDifference {
            elements: Some(Difference {
                plus: elements_plus,
                minus: elements_minus,
            }),
            colored: Some(Difference {
                plus: colored_plus,
                minus: colored_minus,
            }),
            signal_holders: Some(Difference {
                plus: signal_holders_plus,
                minus: signal_holders_minus,
            }),
        }
    }

    pub fn reverse(&self) -> StepDifference {
        let mut elements = None;
        let mut colored = None;
        let mut signal_holders = None;

        if self.elements.is_some() {
            elements = Some(Difference {
                plus: self.elements.clone().unwrap().minus,
                minus: self.elements.clone().unwrap().plus,
            })
        }
        if self.colored.is_some() {
            colored = Some(Difference {
                plus: self.colored.clone().unwrap().minus,
                minus: self.colored.clone().unwrap().plus,
            })
        }
        if self.signal_holders.is_some() {
            signal_holders = Some(Difference {
                plus: self.signal_holders.clone().unwrap().minus,
                minus: self.signal_holders.clone().unwrap().plus,
            })
        }

        StepDifference {
            elements,
            colored,
            signal_holders,
        }
    }
}
