use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::widgets::net_props::Elements;

const SIGN_PLUS: &str = "➕";
const SIGN_MINUS: &str = "➖";

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Difference {
    pub plus: Elements,
    pub minus: Elements,
}

impl Display for Difference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}: {}\n{}: {}",
            SIGN_PLUS, self.plus, SIGN_MINUS, self.minus
        ))
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct StepDifference {
    pub elements: Difference,
    pub colored: Difference,
    pub signal_holders: Difference,
}

impl Display for StepDifference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tooltip_vec = vec![];
        if !self.elements.minus.is_empty() || !self.elements.plus.is_empty() {
            tooltip_vec.push(format!("elements\n{}\n", self.elements));
        }
        if !self.colored.minus.is_empty() || !self.colored.plus.is_empty() {
            tooltip_vec.push(format!("color\n{}\n", self.colored));
        }
        if !self.signal_holders.minus.is_empty() || !self.signal_holders.plus.is_empty() {
            tooltip_vec.push(format!("signal\n{}\n", self.signal_holders));
        }

        f.write_str(tooltip_vec.join("\n").as_str())
    }
}

impl StepDifference {
    pub fn squash(&self, other: &StepDifference) -> StepDifference {
        StepDifference {
            elements: Difference {
                plus: self.elements.plus.union(&other.elements.plus),
                minus: self.elements.minus.union(&other.elements.minus),
            },
            colored: Difference {
                plus: self.colored.plus.union(&other.colored.plus),
                minus: self.colored.minus.union(&other.colored.minus),
            },
            signal_holders: Difference {
                plus: self.signal_holders.plus.union(&other.signal_holders.plus),
                minus: self.signal_holders.minus.union(&other.signal_holders.minus),
            },
        }
    }

    pub fn reverse(&self) -> StepDifference {
        StepDifference {
            elements: Difference {
                plus: self.elements.clone().minus,
                minus: self.elements.clone().plus,
            },
            colored: Difference {
                plus: self.colored.clone().minus,
                minus: self.colored.clone().plus,
            },
            signal_holders: Difference {
                plus: self.signal_holders.clone().minus,
                minus: self.signal_holders.clone().plus,
            },
        }
    }
}
