use std::{collections::HashSet, fmt::Display};

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
    /// Squashes two `StepDifference`s into a single `StepDifference`
    ///
    /// The resulting `StepDifference` will contain the union of the two input `StepDifference`s' elements and selected fields.
    /// Nodes and edges present in both `plus` and `minus` fields will be filtered out.
    ///
    /// # Arguments
    /// * `other` - the `StepDifference` to squash with
    ///
    /// # Returns
    /// A `StepDifference` containing the union of the two `StepDifference`s' elements and selected fields, filtered as described above.
    pub fn squash(&self, other: &StepDifference) -> StepDifference {
        let mut elements_plus = self.elements.plus.union(&other.elements.plus);
        let mut elements_minus = self.elements.minus.union(&other.elements.minus);
        let mut nodes = elements_plus
            .nodes()
            .iter()
            .cloned()
            .filter(|n| elements_minus.nodes_mut().take(n).is_none())
            .collect::<HashSet<_>>();
        let mut edges = elements_plus
            .edges()
            .iter()
            .cloned()
            .filter(|e| elements_minus.edges_mut().take(e).is_none())
            .collect::<HashSet<_>>();
        elements_plus = Elements::new(nodes, edges);

        let mut selected_plus = self.selected.plus.union(&other.selected.plus);
        let mut selected_minus = self.selected.minus.union(&other.selected.minus);
        nodes = selected_plus
            .nodes()
            .iter()
            .cloned()
            .filter(|n| selected_minus.nodes_mut().take(n).is_none())
            .collect::<HashSet<_>>();
        edges = selected_plus
            .edges()
            .iter()
            .cloned()
            .filter(|e| selected_minus.edges_mut().take(e).is_none())
            .collect::<HashSet<_>>();
        selected_plus = Elements::new(nodes, edges);

        StepDifference {
            elements: Difference {
                plus: elements_plus,
                minus: elements_minus,
            },
            selected: Difference {
                plus: selected_plus,
                minus: selected_minus,
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
        }
    }
}
