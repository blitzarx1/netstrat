use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::HashSet, fmt::Debug, fmt::Display, ops::Sub};

use crate::widgets::history::Difference;

use super::{frozen_elements::FrozenElements, ElementID};

const SIGN_NODES: &str = "🇳";
const SIGN_EDGES: &str = "🇪";
const OFFSET: &str = "    ";

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Elements {
    nodes: HashSet<ElementID>,
    edges: HashSet<ElementID>,
    frozen: FrozenElements,
}

// TODO: Don't use Display for such repr. Use caller formatter insted.
impl Display for Elements {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "\n{}{}: {}\n{}{}: {}",
            OFFSET,
            SIGN_NODES,
            self.frozen()
                .nodes()
                .iter()
                .map(|n| n.name.clone())
                .collect::<Vec<_>>()
                .join(", "),
            OFFSET,
            SIGN_EDGES,
            self.frozen()
                .edges()
                .iter()
                .map(|e| e.name.clone())
                .collect::<Vec<_>>()
                .join(", "),
        ))
    }
}

impl Serialize for Elements {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.frozen().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Elements {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let frozen = FrozenElements::deserialize(deserializer)?;

        let nodes = frozen.nodes().iter().cloned().collect::<HashSet<_>>();
        let edges = frozen.edges().iter().cloned().collect::<HashSet<_>>();

        Ok(Elements::new(nodes, edges))
    }
}

impl Elements {
    pub fn new(nodes: HashSet<ElementID>, edges: HashSet<ElementID>) -> Self {
        let mut res = Self {
            nodes,
            edges,
            frozen: Default::default(),
        };

        res.frozen = res.compute_frozen();

        res
    }

    fn compute_frozen(&self) -> FrozenElements {
        let mut nodes = self.nodes().iter().cloned().collect::<Vec<_>>();
        let mut edges = self.edges().iter().cloned().collect::<Vec<_>>();

        nodes.sort();
        edges.sort();

        FrozenElements::new(nodes, edges)
    }

    pub fn apply_difference(&self, diff: Difference) -> Elements {
        let res = self.union(&diff.plus);
        res.sub(&diff.minus)
    }

    /// computes difference which holds set of deleted and added elements
    pub fn compute_difference(&self, other: &Elements) -> Difference {
        let minus_nodes = self
            .nodes
            .iter()
            .filter(|n| !other.nodes.contains(n))
            .cloned()
            .collect::<HashSet<_>>();

        let minus_edges = self
            .edges()
            .iter()
            .filter(|e| !other.edges.contains(e))
            .cloned()
            .collect::<HashSet<_>>();

        let plus_nodes = other
            .nodes
            .iter()
            .filter(|n| !self.nodes.contains(n))
            .cloned()
            .collect::<HashSet<_>>();

        let plus_edges = other
            .edges()
            .iter()
            .filter(|e| !self.edges.contains(e))
            .cloned()
            .collect::<HashSet<_>>();

        Difference {
            plus: Elements::new(plus_nodes, plus_edges),
            minus: Elements::new(minus_nodes, minus_edges),
        }
    }

    pub fn union(&self, other: &Elements) -> Elements {
        let nodes = self.nodes.union(&other.nodes).cloned().collect();
        let edges = self.edges.union(&other.edges).cloned().collect();

        Elements::new(nodes, edges)
    }

    pub fn sub(&self, other: &Elements) -> Elements {
        let nodes = self.nodes.sub(&other.nodes);
        let edges = self.edges.sub(&other.edges);

        Elements::new(nodes, edges)
    }

    pub fn intersection(&self, other: &Elements) -> Elements {
        let nodes = self
            .nodes
            .intersection(&other.nodes)
            .cloned()
            .collect::<HashSet<_>>();
        let edges = self
            .edges
            .intersection(&other.edges)
            .cloned()
            .collect::<HashSet<_>>();

        Elements::new(nodes, edges)
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }

    pub fn nodes(&self) -> &HashSet<ElementID> {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut HashSet<ElementID> {
        &mut self.nodes
    }

    pub fn edges(&self) -> &HashSet<ElementID> {
        &self.edges
    }

    pub fn edges_mut(&mut self) -> &mut HashSet<ElementID> {
        &mut self.edges
    }

    pub fn frozen(&self) -> &FrozenElements {
        &self.frozen
    }
}

#[cfg(test)]
mod test {
    use uuid::Uuid;

    use crate::widgets::net_props::graph::elements::{Edge, Node};

    use super::*;

    const SERIALIZED_DATA: &str = r#"{"nodes":[{"id":"788aa271-f148-48b5-bf79-486071424ccc","name":"fin_3"},{"id":"8ff510fa-c034-40b5-8b82-867c8012bc47","name":"ini_1"},{"id":"a647c909-d020-4cdc-998d-292e2869152e","name":"2"}],"edges":[{"id":"54b2f477-4fad-4cf7-9c10-b7211be19872","name":"2 -> fin_3"},{"id":"ce02dd19-0297-460a-877d-40b62c745b0c","name":"ini_1 -> 2"}]}"#;

    fn elements() -> Elements {
        let n1 = Node::new_with_id(
            Uuid::parse_str("788aa271-f148-48b5-bf79-486071424ccc").unwrap(),
            "fin_3".to_string(),
        );
        let n2 = Node::new_with_id(
            Uuid::parse_str("8ff510fa-c034-40b5-8b82-867c8012bc47").unwrap(),
            "ini_1".to_string(),
        );
        let n3 = Node::new_with_id(
            Uuid::parse_str("a647c909-d020-4cdc-998d-292e2869152e").unwrap(),
            "2".to_string(),
        );
        let nodes = [n1.id().clone(), n2.id().clone(), n3.id().clone()]
            .iter()
            .cloned()
            .collect::<HashSet<_>>();

        let e1 = Edge::new_with_id(
            Uuid::parse_str("54b2f477-4fad-4cf7-9c10-b7211be19872").unwrap(),
            n3.id(),
            n1.id(),
            1.0,
        );
        let e2 = Edge::new_with_id(
            Uuid::parse_str("ce02dd19-0297-460a-877d-40b62c745b0c").unwrap(),
            n2.id(),
            n3.id(),
            1.0,
        );
        let edges = [e1.id().clone(), e2.id().clone()]
            .iter()
            .cloned()
            .collect::<HashSet<_>>();

        Elements::new(nodes, edges)
    }

    #[test]
    fn test_serialize() {
        let elements = elements();

        let res = serde_json::to_string(&elements).unwrap();
        assert_eq!(res, SERIALIZED_DATA);
    }

    #[test]
    fn test_deserialize() {
        let res: Elements = serde_json::from_str(SERIALIZED_DATA).unwrap();

        assert_eq!(res, elements());
    }

    #[test]
    fn test_compute_difference() {
        let els = elements();

        let n1 = Node::new_with_id(
            Uuid::parse_str("788aa271-f148-48b5-bf79-486071424ccc").unwrap(),
            "fin_3".to_string(),
        );
        let els_minus_n1 = els.sub(&Elements::new(
            vec![n1.id().clone()].into_iter().collect(),
            Default::default(),
        ));

        assert_eq!(
            Difference {
                plus: Default::default(),
                minus: Elements::new(
                    vec![n1.id().clone()].into_iter().collect(),
                    Default::default()
                ),
            },
            els.compute_difference(&els_minus_n1)
        );
        assert_eq!(
            Difference {
                plus: Default::default(),
                minus: els.clone(),
            },
            els.compute_difference(&Default::default())
        );
        assert_eq!(
            Difference {
                plus: els.clone(),
                minus: Default::default(),
            },
            Elements::default().compute_difference(&els)
        );
    }
}
