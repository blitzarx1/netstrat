use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::widgets::net_props::Elements;

const SIGN_PLUS: &str = "➕";
const SIGN_MINUS: &str = "➖";

#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
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

impl Difference {
    pub fn is_empty(&self) -> bool {
        self.minus.is_empty() && self.plus.is_empty()
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct StepDifference {
    pub elements: Difference,
    pub selected: Difference,
}

impl Display for StepDifference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tooltip_vec = vec![];
        if !self.elements.minus.is_empty() || !self.elements.plus.is_empty() {
            tooltip_vec.push(format!("elements\n{}\n", self.elements));
        }
        if !self.selected.minus.is_empty() || !self.selected.plus.is_empty() {
            tooltip_vec.push(format!("selected\n{}\n", self.selected));
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
            selected: Difference {
                plus: self.selected.plus.union(&other.selected.plus),
                minus: self.selected.minus.union(&other.selected.minus),
            },
        }
    }

    pub fn reverse(&self) -> StepDifference {
        StepDifference {
            elements: Difference {
                plus: self.elements.clone().minus,
                minus: self.elements.clone().plus,
            },
            selected: Difference {
                plus: self.selected.clone().minus,
                minus: self.selected.clone().plus,
            },
            // signal_holders: Difference {
            //     plus: self.signal_holders.clone().minus,
            //     minus: self.signal_holders.clone().plus,
            // },
        }
    }
}
