use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Node {
    id: Uuid,
    name: String,
    deleted: bool,
}

impl Node {
    pub fn new(name: String) -> Node {
        Node {
            id: Uuid::new_v4(),
            name,
            deleted: false,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn set_name(&self, new_name: String) {
        self.name = new_name;
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Edge {
    id: Uuid,
    weight_x_10_6: i32,
    start: Uuid,
    end: Uuid,
    name: String,
    deleted: bool,
}

impl Edge {
    pub fn new(start: &Node, end: &Node, weight: f64) -> Edge {
        let million = 1_000_000_f64;
        let weight_x_10_6 = (weight * million) as i32;

        // otherwise hack with multiplying by million does not work
        assert!(weight < million);

        let name = [start.name().clone(), end.name().clone()].join(" -> ");

        Edge {
            weight_x_10_6,
            name,
            id: Uuid::new_v4(),
            start: start.id(),
            end: end.id(),
            deleted: false,
        }
    }

    pub fn weight(&self) -> f64 {
        self.weight_x_10_6 as f64 / 1_000_000_f64
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

mod test {
    use super::*;

    #[test]
    fn test_edge_weight() {
        let w = 1.234567;
        let start = Uuid::new_v4();
        let end = Uuid::new_v4();
        let n1 = &Node::new("n1".to_string());
        let n2 = &Node::new("n2".to_string());

        assert_eq!(w, Edge::new(n1, n2, w).weight())
    }
}
